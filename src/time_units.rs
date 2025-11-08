use std::time::Duration;

/// A trait to provide fluent, human-readable Duration creation.
pub trait FluentDuration {
    /// Creates a Duration in seconds.
    fn seconds(self) -> Duration;

    /// Creates a Duration in minutes.
    fn minutes(self) -> Duration;

    /// Creates a Duration in hours.
    fn hours(self) -> Duration;
}

/// Implement FluentDuration for common unsigned integer types.
macro_rules! impl_fluent_duration {
    ($($t:ty),*) => {
        $(
          impl FluentDuration for $t {
            fn seconds(self)->Duration{
              Duration::from_secs(self as u64)
            }

            fn minutes(self)->Duration{
              Duration::from_secs(self as u64 * 60)
            }

            fn hours(self)->Duration{
              Duration::from_secs(self as u64 * 60 * 60)
            }
          }
        )*
    };
}

// Apply the implementation to u32, u64, and usize.
impl_fluent_duration!(u32, u64, usize);
