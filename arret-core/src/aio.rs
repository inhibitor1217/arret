use crate::{error::Result, rate_limiter::AcquireResult};

/// A rate limiter for a single resource, which allows asynchronous
/// requests to be made.
#[async_trait::async_trait]
pub trait RateLimiter {
    /// Try to acquire `tokens` request for the given `resource`.
    ///
    /// If the rate limit has not been exceeded, the resource is acquired and
    /// [`AcquireResult::Ok`] is returned.
    /// Otherwise, [`AcquireResult::Throttled`] is returned.
    ///
    /// Requires a Redis connection to be passed in.
    async fn acquire(
        &self,
        resource: &str,
        tokens: u64,
        con: &mut dyn redis::aio::ConnectionLike,
    ) -> Result<AcquireResult>;
}
