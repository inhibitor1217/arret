use crate::{error::Result, interval::Interval, rate_limiter::RateLimiter};

/// [Fixed window](https://developer.redis.com/develop/java/spring/rate-limiting/fixed-window/)
/// is a simple algorithm for rate limiting. It allows a limited amount of traffic in a fixed
/// time window. Once the window is full, no more traffic is allowed until the window is reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedWindow {
    capacity: u64,
    window: Interval,
}

impl FixedWindow {
    /// Creates a new [`FixedWindow`] with the given capacity and window.
    pub fn new(capacity: u64, window: Interval) -> Result<Self> {
        Ok(Self { capacity, window })
    }

    /// Returns the capacity of the fixed window rule.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns the window of the fixed window rule.
    pub fn window(&self) -> Interval {
        self.window
    }
}

impl RateLimiter for FixedWindow {
    fn acquire(
        &self,
        _resource: &str,
        _tokens: u64,
        _con: &mut dyn redis::ConnectionLike,
    ) -> Result<crate::rate_limiter::AcquireResult> {
        todo!()
    }
}
