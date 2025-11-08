use crate::error::SchedulerError;
use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveTime, Weekday};
use std::collections::HashSet;
use std::time::Duration;

// Type alias for the boxed task function
type Task = Box<dyn FnMut() + Send + 'static>;

/// Represents the set of rules for a schedule
#[derive(Default)]
struct ScheduleRules {
    interval: Option<Duration>,
    at_time: Option<NaiveTime>,
    days_of_week: HashSet<Weekday>,
}

/// A configurable Job that can be added to a Scheduler.
///
/// This struct uses a builder pattern to define a schedule.
///
/// # Examples
///
/// ```
/// use fluent_schedule::{Job, FluentDuration};
/// use chrono::Weekday;
///
/// // A job that runs every 5 minutes
/// let job1 = Job::new()
///     .every(5u32.minutes())
///     .run(|| println!("Five minutes passed!"));
///
/// // A job that runs at 9:30 AM on Mondays
/// let job2 = Job::new()
///     .on(Weekday::Mon)
///     .at("09:30")
///     .run(|| println!("Monday meeting!"));
/// ```
pub struct Job {
    rules: ScheduleRules,
    pub(crate) task: Option<Task>,
    pub(crate) next_run: DateTime<Local>,
    build_error: Option<SchedulerError>,
}

impl Job {
    /// Creates a new, empty job.
    /// This is the entry point for the builder pattern.
    pub fn new() -> Self {
        Self {
            rules: ScheduleRules::default(),
            task: None,
            // Default to now, will be recalculated when .run() is called
            next_run: Local::now(),
            build_error: None,
        }
    }

    /// Schedules the job to run at a fixed interval.
    ///
    /// This is incompatible with `.at()` and day-based scheduling.
    /// If `.at()` was set, it will be cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_schedule::{Job, FluentDuration};
    /// let job = Job::new().every(10u32.seconds());
    /// ```
    pub fn every(mut self, interval: Duration) -> Self {
        // Do nothing if an error is already present
        if self.build_error.is_some() {
            return self;
        }

        self.rules.interval = Some(interval);
        // Interval jobs are incompatible with specific-time jobs
        self.rules.at_time = None;
        self.rules.days_of_week.clear();
        self
    }

    /// Schedules the job to run at a specific time of day.
    ///
    /// This is incompatible with `.every()`.
    /// If `.every()` was set, it will be cleared.
    ///
    /// This function no longer panics. If the time format is invalid,
    /// the error is stored and will be returned when `Scheduler::add()` is called.
    ///
    /// # Arguments
    ///
    /// * `time_str` - A string representing the time, e.g., `"17:00"` or `"09:30:45"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_schedule::Job;
    /// let job = Job::new().at("17:00"); // 5:00 PM
    /// ```
    pub fn at(mut self, time_str: &str) -> Self {
        // Do nothing if an error is already present
        if self.build_error.is_some() {
            return self;
        }

        match NaiveTime::parse_from_str(time_str, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(time_str, "%H:%M:%S"))
        {
            Ok(time) => {
                self.rules.at_time = Some(time);
                // Specific-time jobs are incompatible with interval jobs
                self.rules.interval = None;
            }
            Err(_) => {
                // Store the error instead of panicking
                self.build_error = Some(SchedulerError::InvalidTimeFormat(time_str.to_string()));
            }
        }
        self
    }

    /// Adds a specific day of the week for the job to run.
    /// This is most useful when combined with `.at()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_schedule::Job;
    /// use chrono::Weekday;
    ///
    /// // Run at 10:00 on Monday and Friday
    /// let job = Job::new()
    ///     .on(Weekday::Mon)
    ///     .on(Weekday::Fri)
    ///     .at("10:00");
    /// ```
    pub fn on(mut self, day: Weekday) -> Self {
        self.rules.days_of_week.insert(day);
        self
    }

    /// A helper to schedule a job for all weekdays (Mon-Fri).
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_schedule::Job;
    /// // Run at 5:00 PM every weekday
    /// let job = Job::new().on_weekday().at("17:00");
    /// ```
    pub fn on_weekday(self) -> Self {
        self.on(Weekday::Mon)
            .on(Weekday::Tue)
            .on(Weekday::Wed)
            .on(Weekday::Thu)
            .on(Weekday::Fri)
    }

    /// A helper to schedule a job for the weekend (Sat-Sun).
    ///
    /// # Examples
    ///
    /// ```
    /// use fluent_schedule::Job;
    /// // Run at 10:00 AM on weekends
    /// let job = Job::new().on_weekend().at("10:00");
    /// ```
    pub fn on_weekend(self) -> Self {
        self.on(Weekday::Sat).on(Weekday::Sun)
    }

    /// Sets the task to be executed.
    /// This finalizes the Job configuration and calculates its first run time.
    ///
    /// # Arguments
    ///
    /// * `task` - A closure (`FnMut`) that will be executed.
    ///
    pub fn run<F>(mut self, task: F) -> Self
    where
        F: FnMut() + Send + 'static,
    {
        // Do nothing if an error is already present
        if self.build_error.is_some() {
            return self;
        }

        self.task = Some(Box::new(task));
        // Calculate the first run time
        self.next_run = self.calculate_next_run(Local::now());
        self
    }

    /// Internal: Gets the task closure, consuming it.
    pub(crate) fn take_task(&mut self) -> Option<Task> {
        self.task.take()
    }

    /// Internal: Checks the job for configuration errors.
    pub(crate) fn check_for_errors(&self) -> Result<(), SchedulerError> {
        if let Some(err) = &self.build_error {
            return Err(err.clone());
        }
        if self.task.is_none() {
            return Err(SchedulerError::TaskNotSet);
        }
        Ok(())
    }

    /// Internal: Calculates the next execution time based on the rules.
    pub(crate) fn calculate_next_run(&self, from: DateTime<Local>) -> DateTime<Local> {
        // Case 1: Interval-based job
        if let Some(interval) = self.rules.interval {
            // This is simple: just add the interval
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

            // If day constraints exist, find the next valid day
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
    /// Creates a new, empty job.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time_units::FluentDuration;
    use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveTime, TimeZone, Weekday};

    // Helper: Create a fixed "now" for predictable tests.
    // This date (2025-11-10) is a Monday at 12:00:00
    fn fixed_now() -> DateTime<Local> {
        Local.with_ymd_and_hms(2025, 11, 10, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_job_new_and_default() {
        let job_new = Job::new();
        assert!(job_new.rules.interval.is_none());
        assert!(job_new.rules.at_time.is_none());
        assert!(job_new.rules.days_of_week.is_empty());
        assert!(job_new.task.is_none());
        assert!(job_new.build_error.is_none());

        let job_default = Job::default();
        assert!(job_default.rules.interval.is_none());
    }

    #[test]
    fn test_job_builder_every() {
        let job = Job::new().every(10u32.minutes());
        assert_eq!(job.rules.interval, Some(10u32.minutes()));
        assert!(job.rules.at_time.is_none()); // .every() should clear .at()
    }

    #[test]
    fn test_job_builder_at() {
        let job = Job::new().at("10:30");
        assert_eq!(
            job.rules.at_time,
            Some(NaiveTime::from_hms_opt(10, 30, 0).unwrap())
        );
        assert!(job.rules.interval.is_none()); // .at() should clear .every()
        assert!(job.build_error.is_none());

        let job_secs = Job::new().at("10:30:45");
        assert_eq!(
            job_secs.rules.at_time,
            Some(NaiveTime::from_hms_opt(10, 30, 45).unwrap())
        );
    }

    // --- THIS IS THE CRITICAL CHANGE ---
    // --- DELETE the #[should_panic] test ---
    // #[test]
    // #[should_panic(expected = "Invalid time format")]
    // fn test_job_builder_at_invalid_panic() {
    //     Job::new().at("not-a-time");
    // }

    // --- ADD THIS TEST INSTEAD ---
    #[test]
    fn test_job_builder_at_invalid_no_panic() {
        let job = Job::new().at("not-a-time");
        assert!(job.build_error.is_some());
        assert_eq!(
            job.build_error.as_ref().unwrap(),
            &SchedulerError::InvalidTimeFormat("not-a-time".to_string())
        );

        // Subsequent calls should be ignored
        let job2 = job.every(10u32.seconds());
        assert!(job2.build_error.is_some()); // Error should persist
        assert!(job2.rules.interval.is_none()); // .every() should not have run
    }

    // --- (rest of tests are the same) ---

    #[test]
    fn test_job_builder_on_days() {
        let job = Job::new().on(Weekday::Mon).on(Weekday::Fri);
        assert!(job.rules.days_of_week.contains(&Weekday::Mon));
        assert!(job.rules.days_of_week.contains(&Weekday::Fri));
        assert!(!job.rules.days_of_week.contains(&Weekday::Tue));
    }

    #[test]
    fn test_job_builder_on_weekday() {
        let job = Job::new().on_weekday();
        assert!(job.rules.days_of_week.contains(&Weekday::Mon));
        assert!(job.rules.days_of_week.contains(&Weekday::Tue));
        assert!(job.rules.days_of_week.contains(&Weekday::Wed));
        assert!(job.rules.days_of_week.contains(&Weekday::Thu));
        assert!(job.rules.days_of_week.contains(&Weekday::Fri));
        assert!(!job.rules.days_of_week.contains(&Weekday::Sat));
        assert!(!job.rules.days_of_week.contains(&Weekday::Sun));
    }

    #[test]
    fn test_job_builder_on_weekend() {
        let job = Job::new().on_weekend();
        assert!(!job.rules.days_of_week.contains(&Weekday::Mon));
        assert!(job.rules.days_of_week.contains(&Weekday::Sat));
        assert!(job.rules.days_of_week.contains(&Weekday::Sun));
    }

    #[test]
    fn test_job_run_and_take_task() {
        let mut job = Job::new().every(1u32.seconds()).run(|| println!("test"));
        assert!(job.task.is_some());

        let task = job.take_task();
        assert!(task.is_some());
        assert!(job.task.is_none()); // Task should be gone
    }

    #[test]
    fn test_job_run_with_build_error() {
        // Job has an error
        let job = Job::new().at("bad-time").run(|| {});
        // .run() should not have set the task
        assert!(job.task.is_none());
        assert!(job.build_error.is_some());
    }

    // --- Tests for calculate_next_run ---

    #[test]
    fn test_calc_interval() {
        let now = fixed_now();
        let job = Job::new().every(5u32.minutes());
        let next = job.calculate_next_run(now);
        assert_eq!(next, now + ChronoDuration::try_minutes(5).unwrap());
    }

    #[test]
    fn test_calc_at_today_future() {
        let now = fixed_now(); // 12:00
        let job = Job::new().at("14:00");
        let next = job.calculate_next_run(now);

        let expected_time = NaiveTime::from_hms_opt(14, 0, 0).unwrap();
        assert_eq!(next.date_naive(), now.date_naive()); // Same day
        assert_eq!(next.time(), expected_time);
    }

    #[test]
    fn test_calc_at_today_past() {
        let now = fixed_now(); // 12:00
        let job = Job::new().at("10:00"); // 10:00 is in the past
        let next = job.calculate_next_run(now);

        let expected_time = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let expected_date = now.date_naive() + ChronoDuration::try_days(1).unwrap(); // Tomorrow
        assert_eq!(next.date_naive(), expected_date);
        assert_eq!(next.time(), expected_time);
    }

    #[test]
    fn test_calc_at_on_day_future() {
        let now = fixed_now(); // Monday 12:00
        let job = Job::new().on(Weekday::Mon).at("15:00"); // Today, in the future
        let next = job.calculate_next_run(now);

        assert_eq!(next.date_naive(), now.date_naive()); // Today
        assert_eq!(next.weekday(), Weekday::Mon);
        assert_eq!(next.time(), NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn test_calc_at_on_day_past() {
        let now = fixed_now(); // Monday 12:00
        let job = Job::new().on(Weekday::Mon).at("11:00"); // Today, but in the past
        let next = job.calculate_next_run(now);

        let expected_date = now.date_naive() + ChronoDuration::try_weeks(1).unwrap(); // Next Monday
        assert_eq!(next.date_naive(), expected_date);
        assert_eq!(next.weekday(), Weekday::Mon);
        assert_eq!(next.time(), NaiveTime::from_hms_opt(11, 0, 0).unwrap());
    }

    #[test]
    fn test_calc_at_on_other_day() {
        let now = fixed_now(); // Monday 12:00
        let job = Job::new().on(Weekday::Wed).at("14:00"); // This coming Wednesday
        let next = job.calculate_next_run(now);

        let expected_date = now.date_naive() + ChronoDuration::try_days(2).unwrap(); // This Wed
        assert_eq!(next.date_naive(), expected_date);
        assert_eq!(next.weekday(), Weekday::Wed);
        assert_eq!(next.time(), NaiveTime::from_hms_opt(14, 0, 0).unwrap());
    }

    #[test]
    fn test_calc_no_schedule() {
        let now = fixed_now();
        // A job with a task but no schedule rules
        let job = Job::new().run(|| {});
        let next = job.calculate_next_run(now);

        // Should be scheduled for 1 year in the future (effectively disabled)
        assert_eq!(next, now + ChronoDuration::try_days(365).unwrap());
    }

    #[test]
    fn test_job_check_for_errors() {
        // No error
        let job_ok = Job::new().every(1u32.seconds()).run(|| {});
        assert!(job_ok.check_for_errors().is_ok());

        // Invalid time error
        let job_err_time = Job::new().at("bad-time").run(|| {});
        let err = job_err_time.check_for_errors().unwrap_err();
        assert_eq!(
            err,
            SchedulerError::InvalidTimeFormat("bad-time".to_string())
        );

        // No task error
        let job_err_task = Job::new().every(1u32.seconds());
        let err = job_err_task.check_for_errors().unwrap_err();
        assert_eq!(err, SchedulerError::TaskNotSet);
    }
}
