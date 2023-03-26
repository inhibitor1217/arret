use crate::{
    error::{Error, Result},
    interval::Interval,
    rate_limiter::{AcquireResult, Quota, RateLimiter},
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

    /// Returns the refill amount of the token bucket rule.
    pub fn refill_amount(&self) -> u64 {
        self.refill_amount
    }
}

impl TokenBucket {
    const REDIS_SCRIPT: &str = include_str!("./res/TokenBucket.lua");
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

/// Result of a token bucket Lua script execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TokenBucketScriptResult {
    accepted: bool,
    tokens: u64,
    reset: u64,
}

impl redis::FromRedisValue for TokenBucketScriptResult {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let (ok, tokens, reset): (bool, u64, u64) = redis::FromRedisValue::from_redis_value(v)?;
        Ok(Self {
            accepted: ok,
            tokens,
            reset,
        })
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
