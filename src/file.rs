//! File logger.
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use slog::{Drain, FnValue, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, FullFormat, PlainDecorator};

use {Build, Config, Result};
use misc::{module_and_line, timezone_to_timestamp_fn};
use types::{Format, Severity, SourceLocation, TimeZone};

/// A logger builder which build loggers that write log records to the specified file.
///
/// The resulting logger will work asynchronously (the default channel size is 1024).
#[derive(Debug)]
pub struct FileLoggerBuilder {
    format: Format,
    source_location: SourceLocation,
    timezone: TimeZone,
    level: Severity,
    appender: FileAppender,
    channel_size: usize,
}
impl FileLoggerBuilder {
    /// Makes a new `FileLoggerBuilder` instance.
    ///
    /// This builder will create a logger which uses `path` as
    /// the output destination of the log records.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileLoggerBuilder {
            format: Format::default(),
            source_location: SourceLocation::default(),
            timezone: TimeZone::default(),
            level: Severity::default(),
            appender: FileAppender::new(path),
            channel_size: 1024,
        }
    }

    /// Sets the format of log records.
    pub fn format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }

    /// Sets the source code location type this logger will use.
    pub fn source_location(&mut self, source_location: SourceLocation) -> &mut Self {
        self.source_location = source_location;
        self
    }

    /// Sets the time zone which this logger will use.
    pub fn timezone(&mut self, timezone: TimeZone) -> &mut Self {
        self.timezone = timezone;
        self
    }

    /// Sets the log level of this logger.
    pub fn level(&mut self, severity: Severity) -> &mut Self {
        self.level = severity;
        self
    }

    /// Sets the size of the asynchronous channel of this logger.
    pub fn channel_size(&mut self, channel_size: usize) -> &mut Self {
        self.channel_size = channel_size;
        self
    }

    /// By default, logger just appends log messages to file.
    /// If this method called, logger truncates the file to 0 length when opening.
    pub fn truncate(&mut self) -> &mut Self {
        self.appender.truncate = true;
        self
    }

    fn build_with_drain<D>(&self, drain: D) -> Logger
    where
        D: Drain + Send + 'static,
        D::Err: Debug,
    {
        let drain = Async::new(drain.fuse())
            .chan_size(self.channel_size)
            .build()
            .fuse();

        let drain = self.level.set_level_filter(drain).fuse();

        match self.source_location {
            SourceLocation::None => Logger::root(drain, o!()),
            SourceLocation::ModuleAndLine => {
                Logger::root(drain, o!("module" => FnValue(module_and_line)))
            }
        }
    }
}
impl Build for FileLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let decorator = PlainDecorator::new(self.appender.clone());
        let timestamp = timezone_to_timestamp_fn(self.timezone);
        let logger = match self.format {
            Format::Full => {
                let format = FullFormat::new(decorator).use_custom_timestamp(timestamp);
                self.build_with_drain(format.build())
            }
            Format::Compact => {
                let format = CompactFormat::new(decorator).use_custom_timestamp(timestamp);
                self.build_with_drain(format.build())
            }
        };
        Ok(logger)
    }
}

#[derive(Debug)]
struct FileAppender {
    path: PathBuf,
    file: Option<File>,
    truncate: bool,
}
impl Clone for FileAppender {
    fn clone(&self) -> Self {
        FileAppender {
            path: self.path.clone(),
            file: None,
            truncate: self.truncate,
        }
    }
}
impl FileAppender {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        FileAppender {
            path: path.as_ref().to_path_buf(),
            file: None,
            truncate: false,
        }
    }
    fn reopen_if_needed(&mut self) -> io::Result<()> {
        if !self.path.exists() || self.file.is_none() {
            let mut file_builder = OpenOptions::new();
            file_builder.create(true);
            if self.truncate {
                file_builder.truncate(true);
            }
            let file = file_builder
                .append(!self.truncate)
                .write(true)
                .open(&self.path)?;
            self.file = Some(file);
        }
        Ok(())
    }
}
impl Write for FileAppender {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.reopen_if_needed()?;
        if let Some(ref mut f) = self.file {
            f.write(buf)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Cannot open file: {:?}", self.path),
            ))
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut f) = self.file {
            f.flush()?;
        }
        Ok(())
    }
}

/// The configuration of `FileLoggerBuilder`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileLoggerConfig {
    /// Log level.
    #[serde(default)]
    pub level: Severity,

    /// Log record format.
    #[serde(default)]
    pub format: Format,

    /// Source code location
    #[serde(default)]
    pub source_location: SourceLocation,

    /// Time Zone.
    #[serde(default)]
    pub timezone: TimeZone,

    /// Log file path.
    pub path: PathBuf,

    /// Asynchronous channel size
    #[serde(default = "default_channel_size")]
    pub channel_size: usize,

    /// Truncate the file or not
    #[serde(default)]
    pub truncate: bool,
}
impl Config for FileLoggerConfig {
    type Builder = FileLoggerBuilder;
    fn try_to_builder(&self) -> Result<Self::Builder> {
        let mut builder = FileLoggerBuilder::new(&self.path);
        builder.level(self.level);
        builder.format(self.format);
        builder.source_location(self.source_location);
        builder.timezone(self.timezone);
        builder.channel_size(self.channel_size);
        if self.truncate {
            builder.truncate();
        }
        Ok(builder)
    }
}

fn default_channel_size() -> usize {
    1024
}
