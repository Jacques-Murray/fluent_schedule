use chrono::Weekday;
use fluent_schedule::{FluentDuration, Job, Scheduler};

fn main() {
    // Define the tasks
    let job1 = Job::new()
        .every(5u32.seconds())
        .run(|| println!("Task 1: Running every 5 seconds."));

    let job2 = Job::new()
        .on_weekday()
        .at("17:00")
        .run(|| println!("Task 2: Running at 5 PM on weekdays."));

    let job3 = Job::new()
        .on(Weekday::Sun)
        .at("09:30")
        .run(|| println!("Task 3: Running at 9:30 AM on Sunday."));

    // Create a scheduler
    let mut scheduler = Scheduler::new();

    // Add the jobs
    scheduler.add(job1);
    scheduler.add(job2);
    scheduler.add(job3);

    println!("Starting scheduler...");
    // This will block forever
    scheduler.run_forever();
}
