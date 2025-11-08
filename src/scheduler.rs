use crate::job::Job;
use std::thread;
use std::time::Duration;

/// The main scheduler runtime.
/// It holds all jobs and runs them at their scheduled times.
pub struct Scheduler {
    jobs: Vec<Job>,
}

impl Scheduler {
    /// Creates a new, empty Scheduler.
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Adds a configured Job to the scheduler.
    pub fn add(&mut self, job: Job) {
        if job.task.is_none() {
            // Optionally log a warning: job added without a .run() task
            return;
        }
        self.jobs.push(job);
    }

    /// Starts the scheduler and runs it forever in a loop.
    /// This function will block the current thread.
    pub fn run_forever(mut self) {
        if self.jobs.is_empty() {
            return; // No jobs, nothing to do
        }

        loop {
            let now = chrono::Local::now();
            let mut sleep_duration = Duration::from_secs(60); // Default sleep

            for job in &mut self.jobs {
                if now >= job.next_run {
                    // Re-take the task to run it
                    if let Some(mut task) = job.take_task() {
                        // Run the task
                        (task)();
                        // Put the task back for the next run
                        job.task = Some(task);
                    }

                    // Reschedule for the next run
                    job.next_run = job.calculate_next_run(now);
                }

                // Calculate duration until this job's next run
                let wait_for_job = (job.next_run - now).to_std().unwrap_or(Duration::ZERO);

                // Find the shortest wait time for the next loop
                if wait_for_job > Duration::ZERO && wait_for_job < sleep_duration {
                    sleep_duration = wait_for_job;
                }
            }

            // Sleep until the next job is due, or 1 minute max
            thread::sleep(sleep_duration);
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FluentDuration, Job};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_scheduler_new_and_default() {
        let scheduler_new = Scheduler::new();
        assert_eq!(scheduler_new.jobs.len(), 0);

        let scheduler_default = Scheduler::default();
        assert_eq!(scheduler_default.jobs.len(), 0);
    }

    #[test]
    fn test_scheduler_add_valid_job() {
        let mut scheduler = Scheduler::new();
        let job = Job::new().every(1u32.seconds()).run(|| {});
        scheduler.add(job);
        assert_eq!(scheduler.jobs.len(), 1);
    }

    #[test]
    fn test_scheduler_add_job_no_task() {
        let mut scheduler = Scheduler::new();
        // This job has no .run() call
        let job = Job::new().every(1u32.seconds());
        scheduler.add(job);
        // Should not be added
        assert_eq!(scheduler.jobs.len(), 0);
    }

    #[test]
    fn test_scheduler_run_forever_empty() {
        let scheduler = Scheduler::new();
        // We run this is a thread. The `run_forever` method should see
        // no jobs and return immediately.
        let handle = thread::spawn(move || {
            scheduler.run_forever();
            // If it returns, this value is set
            true
        });

        // Wait for the thread to finish
        let result = handle.join().unwrap();
        assert!(result); // Asserts that the thread exited cleanly
    }

    #[test]
    fn test_scheduler_integration_run_job() {
        // Use an Arc<Mutex> to share a counter between this test
        // and the thread running the scheduler.
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // This job will increment the counter
        let job = Job::new().every(1u32.seconds()).run(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });

        let mut scheduler = Scheduler::new();
        scheduler.add(job);

        // Run the scheduler in its own thread
        thread::spawn(move || {
            scheduler.run_forever();
        });

        // Let the scheduler run for 2.5 seconds
        thread::sleep(Duration::from_millis(2500));

        // Check the counter. It should have been incremented twice
        // (once at t=1s, once at t=2s).
        let count = *counter.lock().unwrap();

        // Use a range to account for slight timing variations
        assert!((2..=3).contains(&count), "Count was {}", count);
    }
}
