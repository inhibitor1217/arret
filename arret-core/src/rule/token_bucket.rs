use crate::{
    error::{Error, Result},
    interval::Interval,
    rate_limiter::{AcquireResult, Quota, RateLimiter},
};

#[cfg(feature = "aio")]
use crate::aio;

use super::clock;

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
    const REDIS_SCRIPT: &str = include_str!("../res/TokenBucket.lua");

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

    /// Returns the refill amount of the token bucket rule.
    pub fn refill_amount(&self) -> u64 {
        self.refill_amount
    }
}

impl RateLimiter for TokenBucket {
    fn acquire(
        &self,
        resource: &str,
        tokens: u64,
        con: &mut dyn redis::ConnectionLike,
    ) -> Result<crate::rate_limiter::AcquireResult> {
        let script = redis::Script::new(Self::REDIS_SCRIPT);
        let key = format!("token_bucket:{resource}");
        let result = script
            .key(&key)
            .arg(clock::now())
            .arg(self.capacity)
            .arg(self.refill_interval.as_secs())
            .arg(self.refill_amount)
            .arg(tokens)
            .invoke::<TokenBucketScriptResult>(con)
            .map_err(|err| Error::Internal(err.to_string()))?;

        if result.accepted {
            Ok(AcquireResult::Ok(Quota::new(
                self.capacity,
                result.tokens,
                result.reset,
            )))
        } else {
            Ok(AcquireResult::Throttled(Quota::new(
                self.capacity,
                result.tokens,
                result.reset,
            )))
        }
    }
}

#[cfg(feature = "aio")]
#[async_trait::async_trait]
impl aio::RateLimiter for TokenBucket {
    async fn acquire<C>(&self, resource: &str, tokens: u64, con: &mut C) -> Result<AcquireResult>
    where
        C: redis::aio::ConnectionLike + Send + Sync,
    {
        let script = redis::Script::new(Self::REDIS_SCRIPT);
        let key = format!("token_bucket:{resource}");
        let result = script
            .key(&key)
            .arg(clock::now())
            .arg(self.capacity)
            .arg(self.refill_interval.as_secs())
            .arg(self.refill_amount)
            .arg(tokens)
            .invoke_async::<C, TokenBucketScriptResult>(con)
            .await
            .map_err(|err| Error::Internal(err.to_string()))?;

        if result.accepted {
            Ok(AcquireResult::Ok(Quota::new(
                self.capacity,
                result.tokens,
                result.reset,
            )))
        } else {
            Ok(AcquireResult::Throttled(Quota::new(
                self.capacity,
                result.tokens,
                result.reset,
            )))
        }
    }
}

/// Result of a token bucket Lua script execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenBucketScriptResult {
    accepted: bool,
    tokens: u64,
    reset: u64,
}

impl redis::FromRedisValue for TokenBucketScriptResult {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let (accepted, tokens, reset): (bool, u64, u64) =
            redis::FromRedisValue::from_redis_value(v)?;
        Ok(Self {
            accepted,
            tokens,
            reset,
        })
    }
}
