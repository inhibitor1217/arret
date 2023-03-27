use std::time::SystemTime;

/// Returns the current time in seconds since the Unix epoch.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
