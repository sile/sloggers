//! Terminal logger.
use std::fmt::Debug;
use std::io;
use slog::{self, Drain, FnValue, Logger};
use slog_async::Async;
use slog_term::{self, CompactFormat, FullFormat, PlainDecorator, TermDecorator};

use {Build, Config, Result};
use misc::{module_and_line, timezone_to_timestamp_fn};
use types::{Format, Severity, TimeZone};

/// A logger builder which build loggers that output log records to the terminal.
///
/// The resulting logger will work asynchronously (the default channel size is 1024).
#[derive(Debug)]
pub struct TerminalLoggerBuilder {
    format: Format,
    timezone: TimeZone,
    destination: Destination,
    level: Severity,
    channel_size: usize,
}
impl TerminalLoggerBuilder {
    /// Makes a new `TerminalLoggerBuilder` instance.
    pub fn new() -> Self {
        TerminalLoggerBuilder {
            format: Format::default(),
            timezone: TimeZone::default(),
            destination: Destination::default(),
            level: Severity::default(),
            channel_size: 1024,
        }
    }

    /// Sets the format of log records.
    pub fn format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }

    /// Sets the time zone which this logger will use.
    pub fn timezone(&mut self, timezone: TimeZone) -> &mut Self {
        self.timezone = timezone;
        self
    }

    /// Sets the destination to which log records will be outputted.
    pub fn destination(&mut self, destination: Destination) -> &mut Self {
        self.destination = destination;
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
        Logger::root(drain, o!("module" => FnValue(module_and_line)))
    }
}
impl Default for TerminalLoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
impl Build for TerminalLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let decorator = self.destination.to_decorator();
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

/// The destination to which log records will be outputted.
///
/// # Examples
///
/// The default value:
///
/// ```
/// use sloggers::terminal::Destination;
///
/// assert_eq!(Destination::default(), Destination::Stdout);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Destination {
    /// Standard output.
    Stdout,

    /// Standard error.
    Stderr,
}
impl Default for Destination {
    fn default() -> Self {
        Destination::Stdout
    }
}
impl Destination {
    fn to_decorator(&self) -> Decorator {
        let maybe_term_decorator = match *self {
            Destination::Stdout => TermDecorator::new().stdout().try_build(),
            Destination::Stderr => TermDecorator::new().stderr().try_build(),
        };
        maybe_term_decorator
            .map(Decorator::Term)
            .unwrap_or_else(|| match *self {
                Destination::Stdout => Decorator::PlainStdout(PlainDecorator::new(io::stdout())),
                Destination::Stderr => Decorator::PlainStderr(PlainDecorator::new(io::stderr())),
            })
    }
}

enum Decorator {
    Term(TermDecorator),
    PlainStdout(PlainDecorator<io::Stdout>),
    PlainStderr(PlainDecorator<io::Stderr>),
}
impl slog_term::Decorator for Decorator {
    fn with_record<F>(
        &self,
        record: &slog::Record,
        logger_values: &slog::OwnedKVList,
        f: F,
    ) -> io::Result<()>
    where
        F: FnOnce(&mut slog_term::RecordDecorator) -> io::Result<()>,
    {
        match *self {
            Decorator::Term(ref d) => d.with_record(record, logger_values, f),
            Decorator::PlainStdout(ref d) => d.with_record(record, logger_values, f),
            Decorator::PlainStderr(ref d) => d.with_record(record, logger_values, f),
        }
    }
}

/// The configuration of `TerminalLoggerBuilder`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TerminalLoggerConfig {
    /// Log level.
    #[serde(default)]
    pub level: Severity,

    /// Log record format.
    #[serde(default)]
    pub format: Format,

    /// Time Zone.
    #[serde(default)]
    pub timezone: TimeZone,

    /// Output destination.
    #[serde(default)]
    pub destination: Destination,

    /// Asynchronous channel size
    #[serde(default = "default_channel_size")]
    pub channel_size: usize,
}
impl Config for TerminalLoggerConfig {
    type Builder = TerminalLoggerBuilder;
    fn try_to_builder(&self) -> Result<Self::Builder> {
        let mut builder = TerminalLoggerBuilder::new();
        builder.level(self.level);
        builder.format(self.format);
        builder.timezone(self.timezone);
        builder.destination(self.destination);
        builder.channel_size(self.channel_size);
        Ok(builder)
    }
}

fn default_channel_size() -> usize {
    1024
}
