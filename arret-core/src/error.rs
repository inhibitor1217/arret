use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Time interval with zero duration is not supported.
    ZeroTimeInterval,

    /// Invalid rate limiting rule.
    InvalidRule(String),

    /// Internal error.
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroTimeInterval => {
                write!(f, "Time interval with zero duration is not supported")
            }
            Self::InvalidRule(msg) => write!(f, "Invalid rate limiting rule: {msg}"),
            Self::Internal(err) => write!(f, "Internal error: {err}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}
