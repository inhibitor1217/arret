use crate::error::Result;

/// A rate limiter for a single resource.
pub trait RateLimiter {
    /// Acquires a resource from the rate limiter. The weight of the resource is
    /// specified by `tokens`.
    ///
    /// If the rate limit has not been exceeded, the resource is acquired and
    /// [`AcquireResult::Ok`] is returned.
    /// Otherwise, [`AcquireResult::Throttled`] is returned.
    ///
    /// Requires a Redis connection to be passed in.
    fn acquire(
        &self,
        resource: &str,
        tokens: u64,
        con: &mut dyn redis::ConnectionLike,
    ) -> Result<AcquireResult>;
}

/// A result from a rate limiting request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquireResult {
    /// The request was allowed.
    Ok(Quota),

    /// The request was denied because the rate limit was exceeded.
    Throttled(Quota),
}

/// Metadata about the current rate limiting state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quota {
    /// The maximum amount of resource that can be requested in an interval.
    pub limit: u64,

    /// The amount of resource remaining in the current interval.
    pub remaining: u64,

    /// The amount of resource that has been requested in the current interval.
    pub used: u64,

    /// The epochmillis timestamp when the current interval will reset.
    ///
    /// The client may use this to determine when to retry the request.
    pub reset: u64,
}

impl Quota {
    /// Creates a new [`Quota`] with the given capacity, used tokens, and reset timestamp.
    pub(crate) fn new(capacity: u64, used: u64, reset: u64) -> Self {
        Self {
            limit: capacity,
            remaining: capacity.saturating_sub(used),
            used,
            reset,
        }
    }
}