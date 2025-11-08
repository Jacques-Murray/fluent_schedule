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
