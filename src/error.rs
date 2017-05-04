use std::io;
use log;
use toml;
use trackable::error::{TrackableError, IntoTrackableError};
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};


/// The error type for this crate.
pub type Error = TrackableError<ErrorKind>;


/// A list of error kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// Invalid input.
    Invalid,

    /// Unknown error.
    Other,
}
impl TrackableErrorKind for ErrorKind {}
impl IntoTrackableError<io::Error> for ErrorKind {
    fn into_trackable_error(e: io::Error) -> Error {
        ErrorKind::Other.cause(e)
    }
}
impl IntoTrackableError<toml::de::Error> for ErrorKind {
    fn into_trackable_error(e: toml::de::Error) -> Error {
        ErrorKind::Invalid.cause(e)
    }
}
impl IntoTrackableError<toml::ser::Error> for ErrorKind {
    fn into_trackable_error(e: toml::ser::Error) -> Error {
        ErrorKind::Invalid.cause(e)
    }
}
impl IntoTrackableError<log::SetLoggerError> for ErrorKind {
    fn into_trackable_error(e: log::SetLoggerError) -> Error {
        ErrorKind::Other.cause(e)
    }
}
