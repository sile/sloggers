//! Logger that sends logs to local syslog daemon. Unix-like platforms only.
//! Uses the [POSIX syslog API] via [slog-syslog].
//!
//! [POSIX syslog API]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/closelog.html
//! [slog-syslog]: https://docs.rs/slog-syslog/0.13/slog_syslog/
//!
//! # Concurrency issues
//!
//! POSIX doesn't support opening more than one connection to syslogd at a
//! time. Although it is safe to construct more than one logger using this
//! module at the same time, some of the settings for syslog loggers will be
//! overwritten by the settings for additional syslog loggers created later.
//!
//! For this reason, the following rules should be followed:
//!
//! * Libraries should not use this module or otherwise call
//!   `openlog` unless specifically told to do so by the main application.
//! * An application that uses this module should not cause `openlog` to be
//!   called from anywhere else.
//! * An application should not use this module to construct more than one
//!   `Logger` at the same time, except when constructing a new `Logger` that
//!   is to replace an old one (for instance, if the application is reloading
//!   its configuration files and reinitializing its logging pipeline).
//!
//! Failure to abide by these rules may result in `closelog` being called at
//! the wrong time. This will cause `openlog` settings (application name,
//! syslog facility, and some flags) to be reset, and there may be a delay in
//! processing the next log message after that (because the connection to the
//! syslog server, if applicable, must be reopened).

#![cfg(unix)]

mod builder;
pub use builder::*;

mod config;
pub use config::*;

pub use slog_syslog::{adapter, Facility, Level, Priority, UnknownFacilityError, UnknownLevelError};
