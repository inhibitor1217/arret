use std::time::Duration;

use crate::error::{Error, Result};

/// Represents a time window for specifing a rate limiting [`Rule`](super::rule::Rule).
///
/// Sub-second precision time windows are not supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval(u64);

impl Interval {
    /// Creates a new [`Interval`] with the given number of seconds.
    pub fn from_secs(seconds: u64) -> Result<Self> {
        if seconds > 0 {
            Ok(Self(seconds))
        } else {
            Err(Error::ZeroTimeInterval)
        }
    }

    /// Creates a new [`Interval`] with a given [`Duration`].
    ///
    /// Sub-second precision duration is not supported. Instead, it will be rounded
    /// down to the nearest second.
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use arret_core::{error::Error, interval::Interval};
    ///
    /// assert_eq!(Interval::from_duration(Duration::from_secs(60)), Interval::from_secs(60));
    /// assert_eq!(Interval::from_duration(Duration::from_secs(1)), Interval::from_secs(1));
    /// assert_eq!(Interval::from_duration(Duration::from_millis(100)), Err(Error::ZeroTimeInterval));
    /// assert_eq!(Interval::from_duration(Duration::ZERO), Err(Error::ZeroTimeInterval));
    /// ```
    pub fn from_duration(duration: Duration) -> Result<Self> {
        Self::from_secs(duration.as_secs())
    }

    /// Returns the number of seconds in the interval.
    ///
    /// ```rust
    /// use arret_core::interval::Interval;
    ///
    /// assert_eq!(Interval::from_secs(60).unwrap().as_secs(), 60);
    /// ```
    pub fn as_secs(&self) -> u64 {
        self.0
    }
}

impl From<Interval> for Duration {
    fn from(interval: Interval) -> Self {
        Duration::from_secs(interval.0)
    }
}
