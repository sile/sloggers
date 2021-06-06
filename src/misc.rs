use slog::Fuse;
use slog::{Drain, FnValue, Logger, Record};
use slog_async::Async;
use slog_kvfilter::KVFilter;
use slog_scope;
use slog_stdlog;
use slog_term;
use crate::types::TimeZone;
use crate::{ErrorKind, Result};
use std::io;
use std::path::Path;
use trackable::error::ErrorKindExt;
use crate::types::KVFilterParameters;
use crate::types::{ProcessID, Severity, SourceLocation};

/// Sets the logger for the log records emitted via `log` crate.
pub fn set_stdlog_logger(logger: Logger) -> Result<()> {
    let _guard = slog_scope::set_global_logger(logger);
    track!(slog_stdlog::init().map_err(|e| ErrorKind::Other.cause(e).into()))
}

pub fn module_and_line(record: &Record) -> String {
    format!("{}:{}", record.module(), record.line())
}

pub fn getpid(_record: &Record) -> String {
    format!("{}", std::process::id())
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

/// builds a logger based on given combination options
pub fn build_with_options(
    drain: Fuse<Async>,
    level: Severity,
    kvfilterparameters: &Option<KVFilterParameters>,
    source_location: SourceLocation,
    process_id: ProcessID,
) -> Logger {
    if let Some(ref p) = kvfilterparameters {
        let kvdrain = KVFilter::new(drain, p.severity.as_level())
            .always_suppress_any(p.always_suppress_any.clone())
            .only_pass_any_on_all_keys(p.only_pass_any_on_all_keys.clone())
            .always_suppress_on_regex(p.always_suppress_on_regex.clone())
            .only_pass_on_regex(p.only_pass_on_regex.clone());

        let drain = level.set_level_filter(kvdrain.fuse());

        match (source_location, process_id) {
            (SourceLocation::None, ProcessID(false)) => Logger::root(drain.fuse(), o!()),
            (SourceLocation::FileAndLine, _) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(file_and_line),
                    ),
            ),
            (SourceLocation::LocalFileAndLine, _) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(local_file_and_line),
                    ),
            ),
            (SourceLocation::ModuleAndLine, ProcessID(false)) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(module_and_line),
                    ),
            ),
            (SourceLocation::None, ProcessID(true)) => Logger::root(
                drain.fuse(),
                o!(
                       "pid" => FnValue(getpid),
                    ),
            ),
            (SourceLocation::ModuleAndLine, ProcessID(true)) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(module_and_line),
                       "pid" => FnValue(getpid),
                    ),
            ),
        }
    } else {
        let drain = level.set_level_filter(drain.fuse());

        match (source_location, process_id) {
            (SourceLocation::None, ProcessID(false)) => Logger::root(drain.fuse(), o!()),
            (SourceLocation::FileAndLine, _) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(file_and_line),
                    ),
            ),
            (SourceLocation::LocalFileAndLine, _) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(local_file_and_line),
                    ),
            ),
            (SourceLocation::ModuleAndLine, ProcessID(false)) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(module_and_line),
                    ),
            ),
            (SourceLocation::None, ProcessID(true)) => Logger::root(
                drain.fuse(),
                o!(
                       "pid" => FnValue(getpid),
                    ),
            ),
            (SourceLocation::ModuleAndLine, ProcessID(true)) => Logger::root(
                drain.fuse(),
                o!(
                       "module" => FnValue(module_and_line),
                       "pid" => FnValue(getpid),
                    ),
            ),
        }
    }
}
