use crate::time_units::FluentDuration;
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveTime, Timelike, Weekday};
use std::collections::HashSet;
use std::time::Duration;

// Type alias for the boxed task function.
type Task = Box<dyn FnMut() + Send + 'static>;

/// Represents the set of rules for a schedule.
#[derive(Default)]
struct ScheduleRules {
    interval: Option<Duration>,
    at_time: Option<NaiveTime>,
    days_of_week: HashSet<Weekday>,
}

/// A configurable Job that can be added to a Scheduler.
pub struct Job {
    rules: ScheduleRules,
    task: Option<Task>,
    pub(crate) next_run: DateTime<Local>,
}

impl Job {
    /// Creates a new, empty job.
    pub fn new() -> Self {
        Self {
            rules: ScheduleRules::default(),
            task: None,
            // Default to now, will be recalculated when .run() is called
            next_run: Local::now(),
        }
    }

    /// Schedules the job to run at a fixed interval.
    /// Example: `Job::new().every(5.minutes())`
    pub fn every(mut self, interval: Duration) -> Self {
        self.rules.interval = Some(interval);
        // Interval jobs are incompatible with specific-time jobs
        self.rules.at_time = None;
        self.rules.days_of_week.clear();
        self
    }

    /// Schedules the job to run at a specific time of day.
    /// Expects format "HH:MM" or "HH:MM:SS".
    /// Example: `Job::new().at("17:00")`.
    pub fn at(mut self, time_str: &str) -> Self {
        let time = NaiveTime::parse_from_str(time_str, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M:%S"))
            .expect("Invalid time format. Use 'HH:MM' or 'HH:MM:SS'");

        self.rules.at_time = Some(time);
        // Specific-time jobs are incompatible with interval jobs
        self.rules.interval = None;
        self
    }

    /// Adds a specific day of the week for the job to run.
    /// This is often used with `.at()`.
    /// Example: `Job::new().on(Weekday::Mon).at("09:00")`
    pub fn on(mut self, day: Weekday) -> Self {
        self.rules.days_of_week.insert(day);
        self
    }

    /// A helper to schedule a job for all weekdays.
    pub fn on_weekday(self) -> Self {
        self.on(Weekday::Mon)
            .on(Weekday::Tue)
            .on(Weekday::Wed)
            .on(Weekday::Thu)
            .on(Weekday::Fri)
    }

    /// A helper to schedule a job for the weekend.
    pub fn on_weekend(self) -> Self {
        self.on(Weekday::Sat).on(Weekday::Sun)
    }

    /// Sets the task to be executed. This finalizes the Job configuration.
    pub fn run<F>(mut self, task: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        self.task = Some(Box::new(task));
        // Calculate the first run time
        self.next_run = self.calculate_next_run(Local::now());
        self
    }

    /// Internal: Gets the task closure, consuming it.
    pub(crate) fn take_task(&mut self) -> Option<Task> {
        self.task.take()
    }

    /// Internal: Calculates the next execution time based on the rules.
    pub(crate) fn calculate_next_run(&self, from: DateTime<Local>) -> DateTime<Local> {
        // Case 1: Interval-based job
        if let Some(interval) = self.rules.interval {
            // This is simple: just add the interval
            // We use std::time::Duration, need to convert to chrono::Duration
            let chrono_interval =
                ChronoDuration::from_std(interval).expect("Interval duration is too large");
            return from + chrono_interval;
        }

        // Case 2: Specific-time job
        if let Some(time) = self.rules.at_time {
            let mut next_run = from.date_naive().and_time(time);

            // If the time is already past for today, start from tomorrow
            if from.time() >= time {
                next_run = next_run + ChronoDuration::try_days(1).expect("Duration overflow");
            }

            // If day contraints exist, find the next valid day
            if !self.rules.days_of_week.is_empty() {
                while !self.rules.days_of_week.contains(&next_run.weekday()) {
                    next_run = next_run + ChronoDuration::try_days(1).expect("Duration overflow");
                }
            }

            // Convert back to local DateTime
            return next_run.and_local_timezone(Local).unwrap();
        }

        // Default case (e.g., a job with no schedule): run 1 year from now
        // This effectively disables the job
        from + ChronoDuration::try_days(365).expect("Duration overflow")
    }
}

impl Default for Job {
    fn default() -> Self {
        Self::new()
    }
}
