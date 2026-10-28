use chrono::Weekday;
use fluent_schedule::{FluentDuration, Job, Scheduler, SchedulerError};

#[test]
fn accepts_multiple_schedule_types_without_executing_tasks_during_registration() {
    let mut scheduler = Scheduler::default();
    for job in [
        Job::new().every(5u32.seconds()),
        Job::new().on_weekday().at("09:30"),
        Job::new().on_weekend().at("10:00:15"),
        Job::default().on(Weekday::Mon).on(Weekday::Fri).at("17:00"),
    ] {
        scheduler
            .add(job.run(|| panic!("registration must not execute a task")))
            .unwrap();
    }
}

#[test]
fn invalid_times_preserve_the_original_error_across_builder_calls() {
    for time in ["", "99:99", "24:00", "12:60", "12:30:61", "not-a-time"] {
        let job = Job::new()
            .at(time)
            .every(1u64.minutes())
            .at("09:00")
            .on_weekday()
            .run(|| {});
        assert_eq!(
            Scheduler::new().add(job),
            Err(SchedulerError::InvalidTimeFormat(time.to_owned()))
        );
    }
}

#[test]
fn rejected_jobs_do_not_prevent_subsequent_registration() {
    let mut scheduler = Scheduler::new();
    assert_eq!(
        scheduler.add(Job::new().every(1usize.hours())),
        Err(SchedulerError::TaskNotSet)
    );
    scheduler
        .add(Job::new().every(1u32.seconds()).run(|| {}))
        .unwrap();
}

#[test]
fn scheduler_with_only_rejected_jobs_returns_immediately() {
    let mut scheduler = Scheduler::new();
    assert!(scheduler.add(Job::new().at("invalid").run(|| {})).is_err());
    scheduler.run_forever();
}

#[test]
fn errors_provide_actionable_messages_and_standard_error_support() {
    let error = SchedulerError::InvalidTimeFormat("bad".into());
    assert_eq!(
        error.to_string(),
        "Invalid time format: 'bad'. Expected 'HH:MM' or 'HH:MM:SS'."
    );
    let error: &dyn std::error::Error = &SchedulerError::TaskNotSet;
    assert!(error.to_string().contains(".run()"));
    assert!(error.source().is_none());
}
