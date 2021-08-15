use crate::types::TimeZone;
use crate::{Error, ErrorKind, Result};
use slog::{Logger, Record};
use std::io;
use std::path::Path;
use trackable::error::ErrorKindExt;

/// Sets the logger for the log records emitted via `log` crate.
///
/// # Examples
///
/// ```
/// use sloggers::Build as _;
///
/// # fn main() -> sloggers::Result<()> {
/// let logger = sloggers::terminal::TerminalLoggerBuilder::new().build()?;
/// let _guard = sloggers::set_stdlog_logger(logger.clone())?;
///
/// slog::info!(logger, "Hello ");
/// log::info!("World!");
/// # Ok(())
/// # }
/// ```
pub fn set_stdlog_logger(logger: Logger) -> Result<slog_scope::GlobalLoggerGuard> {
    track!(slog_stdlog::init().map_err(|e| Error::from(ErrorKind::Other.cause(e))))?;
    Ok(slog_scope::set_global_logger(logger))
}

pub fn module_and_line(record: &Record) -> String {
    format!("{}:{}", record.module(), record.line())
}

pub fn file_and_line(record: &Record) -> String {
    format!("{}:{}", record.file(), record.line())
}

pub fn local_file_and_line(record: &Record) -> String {
    if Path::new(record.file()).is_relative() {
        file_and_line(record)
    } else {
        module_and_line(record)
    }
}

pub fn timezone_to_timestamp_fn(timezone: TimeZone) -> fn(&mut dyn io::Write) -> io::Result<()> {
    match timezone {
        TimeZone::Utc => slog_term::timestamp_utc,
        TimeZone::Local => slog_term::timestamp_local,
    }
}
