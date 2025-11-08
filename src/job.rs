use chrono::{DateTime, Datelike, Duration as ChronoDuration, Local, NaiveTime, Weekday};
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
    pub(crate) task: Option<Task>,
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

        let job_secs = Job::new().at("10:30:45");
        assert_eq!(
            job_secs.rules.at_time,
            Some(NaiveTime::from_hms_opt(10, 30, 45).unwrap())
        );
    }

    #[test]
    #[should_panic(expected = "Invalid time format")]
    fn test_job_builder_at_invalid_panic() {
        Job::new().at("not-a-time");
    }

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
        let job = Job::new().at("10:00");
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
}
