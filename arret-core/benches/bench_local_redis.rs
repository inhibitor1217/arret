use arret_core::{
    interval::Interval,
    rate_limiter::RateLimiter,
    rule::{FixedWindow, TokenBucket},
};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};

#[cfg(feature = "aio")]
use arret_core::aio;

fn prepare_connection() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");
    client
        .get_connection()
        .expect("Failed to get Redis connection")
}

#[cfg(feature = "aio")]
async fn prepare_connection_async() -> redis::aio::MultiplexedConnection {
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");
    client
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to get Redis connection")
}

#[cfg(feature = "aio")]
fn current_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
}

fn bench_token_bucket(b: &mut Bencher) {
    let mut con = prepare_connection();
    let token_bucket = TokenBucket::new(1_000, Interval::from_secs(1).unwrap(), 1_000).unwrap();

    b.iter(|| {
        let _ = token_bucket.acquire("bench_token_bucket", 1, &mut con);
    })
}

#[cfg(feature = "aio")]
async fn token_bucket_async(con: &redis::aio::MultiplexedConnection) {
    let mut con = con.clone();

    let token_bucket = TokenBucket::new(1_000, Interval::from_secs(1).unwrap(), 1_000).unwrap();

    let _ = aio::RateLimiter::acquire(&token_bucket, "bench_token_bucket_async", 1, &mut con)
        .await
        .expect("Failed to acquire from token bucket");
}

#[cfg(feature = "aio")]
fn bench_token_bucket_async(b: &mut Bencher) {
    use futures::future;

    let runtime = current_thread_runtime();
    let con = runtime.block_on(prepare_connection_async());

    b.to_async(runtime).iter_custom(|iters| {
        let con = con.clone();

        async move {
            let start = std::time::Instant::now();

            let futures = (0..iters).map(|_| token_bucket_async(&con));
            future::join_all(futures).await;

            start.elapsed()
        }
    })
}

fn bench_fixed_window(b: &mut Bencher) {
    let mut con = prepare_connection();
    let fixed_window = FixedWindow::new(1_000, Interval::from_secs(1).unwrap()).unwrap();

    b.iter(|| {
        let _ = fixed_window.acquire("bench_fixed_window", 1, &mut con);
    })
}

#[cfg(feature = "aio")]
async fn fixed_window_async(con: &redis::aio::MultiplexedConnection) {
    let mut con = con.clone();

    let fixed_window = FixedWindow::new(1_000, Interval::from_secs(1).unwrap()).unwrap();

    let _ = aio::RateLimiter::acquire(&fixed_window, "bench_fixed_window_async", 1, &mut con)
        .await
        .expect("Failed to acquire from fixed window");
}

#[cfg(feature = "aio")]
fn bench_fixed_window_async(b: &mut Bencher) {
    let runtime = current_thread_runtime();

    let con = runtime.block_on(prepare_connection_async());

    b.to_async(runtime).iter_custom(|iters| {
        let con = con.clone();

        async move {
            let start = std::time::Instant::now();

            let futures = (0..iters).map(|_| fixed_window_async(&con));
            futures::future::join_all(futures).await;

            start.elapsed()
        }
    })
}

fn bench_sync_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_query");

    group
        .bench_function("token_bucket", bench_token_bucket)
        .bench_function("fixed_window", bench_fixed_window);
    group.finish();
}

#[cfg(feature = "aio")]
fn bench_async_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("async_query");

    group
        .bench_function("token_bucket", bench_token_bucket_async)
        .bench_function("fixed_window", bench_fixed_window_async);
    group.finish();
}

#[cfg(feature = "aio")]
criterion_group!(benches, bench_sync_query, bench_async_query);
#[cfg(not(feature = "aio"))]
criterion_group!(benches, bench_sync_query);

criterion_main!(benches);
