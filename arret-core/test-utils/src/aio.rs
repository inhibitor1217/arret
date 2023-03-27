use std::future;

fn current_thread_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
}

pub async fn prepare_redis_async_connection() -> redis::aio::Connection {
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");
    client
        .get_async_connection()
        .await
        .expect("Failed to get Redis connection")
}

pub fn block_on<F>(f: F) -> F::Output
where
    F: future::Future,
{
    current_thread_runtime().block_on(f)
}
