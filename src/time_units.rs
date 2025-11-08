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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_fluent_seconds() {
        assert_eq!(1u32.seconds(), Duration::from_secs(1));
        assert_eq!(5u64.seconds(), Duration::from_secs(5));
        assert_eq!(10usize.seconds(), Duration::from_secs(10));
    }

    #[test]
    fn test_fluent_minutes() {
        assert_eq!(1u32.minutes(), Duration::from_secs(60));
        assert_eq!(2u64.minutes(), Duration::from_secs(120));
        assert_eq!(3usize.minutes(), Duration::from_secs(180));
    }

    #[test]
    fn test_fluent_hours() {
        assert_eq!(1u32.hours(), Duration::from_secs(3600));
        assert_eq!(2u64.hours(), Duration::from_secs(7200));
        assert_eq!(1usize.hours(), Duration::from_secs(3600));
    }
}
