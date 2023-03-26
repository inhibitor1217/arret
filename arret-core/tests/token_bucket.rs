use arret_core::{
    interval::Interval,
    rate_limiter::{AcquireResult, Quota, RateLimiter},
    rule::TokenBucket,
};
use test_utils::{assert_ok, assert_throttled, prepare_redis_connection, wait};

#[test]
fn single_token() {
    let mut con = prepare_redis_connection();

    let token_bucket = TokenBucket::new(10, Interval::from_secs(10).unwrap(), 10).unwrap();

    let res = token_bucket
        .acquire("res:single_token", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_ok!(res, 10, 9);
}

#[test]
fn multiple_token() {
    let mut con = prepare_redis_connection();

    let token_bucket = TokenBucket::new(10, Interval::from_secs(10).unwrap(), 10).unwrap();

    let res = token_bucket
        .acquire("res:multiple_token", 5, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_ok!(res, 10, 5);
}

#[test]
fn zero_capacity() {
    let mut con = prepare_redis_connection();

    let token_bucket = TokenBucket::new(0, Interval::from_secs(10).unwrap(), 10).unwrap();

    let res = token_bucket
        .acquire("res:zero_capacity", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_throttled!(res, 0, 0);
}

#[test]
fn throttled() {
    let mut con = prepare_redis_connection();

    let token_bucket = TokenBucket::new(10, Interval::from_secs(10).unwrap(), 10).unwrap();

    for i in 0..5 {
        let res = token_bucket
            .acquire("res:throttled", 3, &mut con)
            .expect("Failed to acquire from token bucket");

        if i < 3 {
            assert_ok!(res, 10, 7 - i * 3);
        } else {
            assert_throttled!(res, 10, 1);
        }
    }
}

#[test]
fn refill() {
    let mut con = prepare_redis_connection();

    let token_bucket = TokenBucket::new(2, Interval::from_secs(1).unwrap(), 1).unwrap();

    let res = token_bucket
        .acquire("res:refill", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_ok!(res, 2, 1);

    let res = token_bucket
        .acquire("res:refill", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_ok!(res, 2, 0);

    let res = token_bucket
        .acquire("res:refill", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_throttled!(res, 2, 0);

    wait(1);

    let res = token_bucket
        .acquire("res:refill", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_ok!(res, 2, 0);

    let res = token_bucket
        .acquire("res:refill", 1, &mut con)
        .expect("Failed to acquire from token bucket");

    assert_throttled!(res, 2, 0);
}
