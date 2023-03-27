use arret_core::{
    interval::Interval,
    rate_limiter::{AcquireResult, Quota, RateLimiter},
    rule::FixedWindow,
};
use test_utils::{assert_ok, assert_throttled, prepare_redis_connection, wait};

#[cfg(feature = "aio")]
use arret_core::aio;

#[cfg(feature = "aio")]
use test_utils::aio::{block_on, prepare_redis_async_connection};

#[test]
fn single_token() {
    let mut con = prepare_redis_connection();

    let fixed_window = FixedWindow::new(10, Interval::from_secs(10).unwrap()).unwrap();

    let res = fixed_window
        .acquire("res:single_token", 1, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_ok!(res, 10, 9);
}

#[cfg(feature = "aio")]
#[test]
fn single_token_async() {
    block_on(async {
        let mut con = prepare_redis_async_connection().await;

        let fixed_window = FixedWindow::new(10, Interval::from_secs(10).unwrap()).unwrap();

        let res = aio::RateLimiter::acquire(&fixed_window, "res:single_token_async", 1, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_ok!(res, 10, 9);
    })
}

#[test]
fn multiple_token() {
    let mut con = prepare_redis_connection();

    let fixed_window = FixedWindow::new(10, Interval::from_secs(10).unwrap()).unwrap();

    let res = fixed_window
        .acquire("res:multiple_token", 5, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_ok!(res, 10, 5);
}

#[cfg(feature = "aio")]
#[test]
fn multiple_token_async() {
    block_on(async {
        let mut con = prepare_redis_async_connection().await;

        let fixed_window = FixedWindow::new(10, Interval::from_secs(10).unwrap()).unwrap();

        let res = aio::RateLimiter::acquire(&fixed_window, "res:multiple_token_async", 5, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_ok!(res, 10, 5);
    })
}

#[test]
fn zero_capacity() {
    let mut con = prepare_redis_connection();

    let fixed_window = FixedWindow::new(0, Interval::from_secs(10).unwrap()).unwrap();

    let res = fixed_window
        .acquire("res:zero_capacity", 1, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_throttled!(res, 0, 0);
}

#[cfg(feature = "aio")]
#[test]
fn zero_capacity_async() {
    block_on(async {
        let mut con = prepare_redis_async_connection().await;

        let fixed_window = FixedWindow::new(0, Interval::from_secs(10).unwrap()).unwrap();

        let res = aio::RateLimiter::acquire(&fixed_window, "res:zero_capacity_async", 1, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_throttled!(res, 0, 0);
    })
}

#[test]
fn throttled() {
    let mut con = prepare_redis_connection();

    let fixed_window = FixedWindow::new(10, Interval::from_secs(10).unwrap()).unwrap();

    for i in 0..5 {
        let res = fixed_window
            .acquire("res:throttled", 3, &mut con)
            .expect("Failed to acquire from fixed window");

        if i < 3 {
            assert_ok!(res, 10, 7 - i * 3);
        } else {
            assert_throttled!(res, 10, 1);
        }
    }
}

#[cfg(feature = "aio")]
#[test]
fn throttled_async() {
    block_on(async {
        let mut con = prepare_redis_async_connection().await;

        let fixed_window = FixedWindow::new(10, Interval::from_secs(10).unwrap()).unwrap();

        for i in 0..5 {
            let res = aio::RateLimiter::acquire(&fixed_window, "res:throttled_async", 3, &mut con)
                .await
                .expect("Failed to acquire from fixed window");

            if i < 3 {
                assert_ok!(res, 10, 7 - i * 3);
            } else {
                assert_throttled!(res, 10, 1);
            }
        }
    })
}

#[test]
fn next_window() {
    let mut con = prepare_redis_connection();

    let fixed_window = FixedWindow::new(2, Interval::from_secs(1).unwrap()).unwrap();

    let res = fixed_window
        .acquire("res:next_window", 1, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_ok!(res, 2, 1);

    let res = fixed_window
        .acquire("res:next_window", 1, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_ok!(res, 2, 0);

    let res = fixed_window
        .acquire("res:next_window", 1, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_throttled!(res, 2, 0);

    wait(1);

    let res = fixed_window
        .acquire("res:next_window", 1, &mut con)
        .expect("Failed to acquire from fixed window");

    assert_ok!(res, 2, 1);
}

#[cfg(feature = "aio")]
#[test]
fn next_window_async() {
    use std::time::Duration;

    block_on(async {
        let mut con = prepare_redis_async_connection().await;

        let fixed_window = FixedWindow::new(2, Interval::from_secs(1).unwrap()).unwrap();

        let res = aio::RateLimiter::acquire(&fixed_window, "res:next_window_async", 1, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_ok!(res, 2, 1);

        let res = aio::RateLimiter::acquire(&fixed_window, "res:next_window_async", 1, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_ok!(res, 2, 0);

        let res = aio::RateLimiter::acquire(&fixed_window, "res:next_window_async", 1, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_throttled!(res, 2, 0);

        tokio::time::sleep(Duration::from_secs(1)).await;

        let res = aio::RateLimiter::acquire(&fixed_window, "res:next_window_async", 1, &mut con)
            .await
            .expect("Failed to acquire from fixed window");

        assert_ok!(res, 2, 1);
    })
}
