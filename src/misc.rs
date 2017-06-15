use std::io;
use slog::{Logger, Record};
use slog_term;
use slog_scope;
use slog_stdlog;

use {Result, Error};
use types::TimeZone;

/// Sets the logger for the log records emitted via `log` crate.
pub fn set_stdlog_logger(logger: Logger) -> Result<()> {
    let _guard = slog_scope::set_global_logger(logger);
    track!(slog_stdlog::init().map_err(Error::from))
}

pub fn module_and_line(record: &Record) -> String {
    format!("{}:{}", record.module(), record.line())
}

pub fn timezone_to_timestamp_fn(timezone: TimeZone) -> fn(&mut io::Write) -> io::Result<()> {
    match timezone {
        TimeZone::Utc => slog_term::timestamp_utc,
        TimeZone::Local => slog_term::timestamp_local,
    }
}
