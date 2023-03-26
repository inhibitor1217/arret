use std::time::Duration;

use crate::{
    error::{Error, Result},
    interval::Interval,
};

/// Rate limiting rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Rate limiting rule based on token bucket algorithm.
    ///
    /// See [`TokenBucket`] for details.
    TokenBucket(TokenBucket),

    /// Rate limiting rule based on fixed window algorithm.
    ///
    /// See [`FixedWindow`] for details.
    FixedWindow(FixedWindow),
}

/// [Token bucket](https://en.wikipedia.org/wiki/Token_bucket) algorithm is a common
/// algorithm for rate limiting. While it allows traffic to be passed at a constant rate,
/// it also allows bursts of traffic to be passed over a short period of time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenBucket {
    capacity: u64,
    refill_interval: Interval,
    refill_amount: u64,
}

impl TokenBucket {
    /// Creates a new [`TokenBucket`] with the given capacity, refill interval and refill amount.
    ///
    /// # Errors
    /// - [`Error::InvalidRule`] if `refill_amount` is zero.
    pub fn new(capacity: u64, refill_interval: Interval, refill_amount: u64) -> Result<Self> {
        if refill_amount == 0 {
            Err(Error::InvalidRule(
                "Refill amount must be greater than zero".into(),
            ))
        } else {
            Ok(Self {
                capacity,
                refill_interval,
                refill_amount,
            })
        }
    }

    /// Returns the capacity of the token bucket rule.
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns the refill interval of the token bucket rule.
    pub fn refill_interval(&self) -> Interval {
        self.refill_interval
    }

    /// Returns the refill interval of the token bucket rule as [`Duration`].
    pub fn refill_duration(&self) -> Duration {
        self.refill_interval.into()
    }

    /// Returns the refill amount of the token bucket rule.
    pub fn refill_amount(&self) -> u64 {
        self.refill_amount
    }
}

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

    /// Returns the window of the fixed window rule as [`Duration`].
    pub fn window_duration(&self) -> Duration {
        self.window.into()
    }
}
