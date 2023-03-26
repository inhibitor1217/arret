use arret_core::{
    interval::Interval,
    rate_limiter::RateLimiter,
    rule::{FixedWindow, TokenBucket},
};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};

fn prepare_connection() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");
    client
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn bench_token_bucket(b: &mut Bencher) {
    let mut con = prepare_connection();
    let token_bucket = TokenBucket::new(1_000, Interval::from_secs(1).unwrap(), 1_000).unwrap();

    b.iter(|| {
        let _ = token_bucket.acquire("bench_token_bucket", 1, &mut con);
    })
}

fn bench_fixed_window(b: &mut Bencher) {
    let mut con = prepare_connection();
    let fixed_window = FixedWindow::new(1_000, Interval::from_secs(1).unwrap()).unwrap();

    b.iter(|| {
        let _ = fixed_window.acquire("bench_fixed_window", 1, &mut con);
    })
}

fn bench_sync_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync_query");

    group
        .bench_function("token_bucket", bench_token_bucket)
        .bench_function("fixed_window", bench_fixed_window);
    group.finish();
}

criterion_group!(benches, bench_sync_query);
criterion_main!(benches);
