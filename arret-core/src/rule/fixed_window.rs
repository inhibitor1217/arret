use crate::{
    error::{Error, Result},
    interval::Interval,
    rate_limiter::{AcquireResult, Quota, RateLimiter},
};

/// [Fixed window](https://developer.redis.com/develop/java/spring/rate-limiting/fixed-window/)
/// is a simple algorithm for rate limiting. It allows a limited amount of traffic in a fixed
/// time window. Once the window is full, no more traffic is allowed until the window is reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedWindow {
    capacity: u64,
    window: Interval,
}

impl FixedWindow {
    const REDIS_SCRIPT: &str = include_str!("../res/FixedWindow.lua");

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
        resource: &str,
        tokens: u64,
        con: &mut dyn redis::ConnectionLike,
    ) -> Result<crate::rate_limiter::AcquireResult> {
        let script = redis::Script::new(Self::REDIS_SCRIPT);

        let (seconds, _): (u64, u64) = redis::cmd("TIME")
            .query(con)
            .map_err(|err| Error::Internal(err.to_string()))?;
        let window_id = seconds / self.window.as_secs();
        let slot = format!("fixed_window:{resource}:{window_id}");
        let reset = (window_id + 1) * self.window.as_secs();

        let result: FixedWindowScriptResult = script
            .key(&slot)
            .arg(self.capacity)
            .arg(self.window.as_secs())
            .arg(tokens)
            .invoke(con)
            .map_err(|err| Error::Internal(err.to_string()))?;

        if result.accepted {
            Ok(AcquireResult::Ok(Quota::new(
                self.capacity,
                result.bucket,
                reset,
            )))
        } else {
            Ok(AcquireResult::Throttled(Quota::new(
                self.capacity,
                result.bucket,
                reset,
            )))
        }
    }
}

/// Result of a fixed window Lua script execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FixedWindowScriptResult {
    accepted: bool,
    bucket: u64,
}

impl redis::FromRedisValue for FixedWindowScriptResult {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        let (accepted, bucket): (bool, u64) = redis::FromRedisValue::from_redis_value(v)?;
        Ok(Self { accepted, bucket })
    }
}
