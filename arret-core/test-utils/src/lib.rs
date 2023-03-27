#[cfg(feature = "aio")]
pub mod aio;

pub fn prepare_redis_connection() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1:6379").expect("Failed to connect to Redis");
    client
        .get_connection()
        .expect("Failed to get Redis connection")
}

pub fn wait(seconds: u64) {
    std::thread::sleep(std::time::Duration::from_secs(seconds));
}

#[macro_export]
macro_rules! assert_ok {
    ($res:expr, $limit:expr, $remaining:expr) => {
        match $res {
            AcquireResult::Ok(Quota {
                limit, remaining, ..
            }) => {
                assert_eq!(limit, $limit);
                assert_eq!(remaining, $remaining);
            }
            _ => panic!("Expected Ok, got {:?}", $res),
        }
    };
}

#[macro_export]
macro_rules! assert_throttled {
    ($res:expr, $limit:expr, $remaining:expr) => {
        match $res {
            AcquireResult::Throttled(Quota {
                limit, remaining, ..
            }) => {
                assert_eq!(limit, $limit);
                assert_eq!(remaining, $remaining);
            }
            _ => panic!("Expected Throttled, got {:?}", $res),
        }
    };
}
