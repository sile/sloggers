use std::fmt::Debug;
use slog::{Logger, Drain, FnValue};
use slog_async::Async;
use slog_term::{TermDecorator, CompactFormat, FullFormat};

use {Result, Build, Config};
use misc::module_and_line;
use types::{Format, Severity, Timezone};

#[derive(Debug)]
pub struct TerminalLoggerBuilder {
    format: Format,
    timezone: Timezone,
    destination: Destination,
    level: Severity,
}
impl TerminalLoggerBuilder {
    pub fn new() -> Self {
        TerminalLoggerBuilder {
            format: Format::default(),
            timezone: Timezone::default(),
            destination: Destination::default(),
            level: Severity::default(),
        }
    }
    pub fn format(&mut self, format: Format) -> &mut Self {
        self.format = format;
        self
    }
    pub fn full(&mut self) -> &mut Self {
        self.format(Format::Full)
    }
    pub fn compact(&mut self) -> &mut Self {
        self.format(Format::Compact)
    }
    pub fn timezone(&mut self, timezone: Timezone) -> &mut Self {
        self.timezone = timezone;
        self
    }
    pub fn utc_time(&mut self) -> &mut Self {
        self.timezone(Timezone::Utc)
    }
    pub fn local_time(&mut self) -> &mut Self {
        self.timezone(Timezone::Local)
    }
    pub fn destination(&mut self, destination: Destination) -> &mut Self {
        self.destination = destination;
        self
    }
    pub fn stdout(&mut self) -> &mut Self {
        self.destination(Destination::Stdout)
    }
    pub fn stderr(&mut self) -> &mut Self {
        self.destination(Destination::Stderr)
    }
    pub fn level(&mut self, severity: Severity) -> &mut Self {
        self.level = severity;
        self
    }
    fn build_with_drain<D>(&self, drain: D) -> Logger
        where D: Drain + Send + 'static,
              D::Err: Debug
    {
        let drain = Async::default(drain.fuse()).fuse();
        let drain = self.level.set_level_filter(drain).fuse();
        Logger::root(drain, o!("module" => FnValue(module_and_line)))
    }
}
impl Build for TerminalLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let decorator = self.destination.to_term_decorator();
        let timestamp = self.timezone.to_timestamp_fn();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Destination {
    #[serde(rename = "stdout")]
    Stdout,

    #[serde(rename = "stderr")]
    Stderr,
}
impl Default for Destination {
    fn default() -> Self {
        Destination::Stdout
    }
}
impl Destination {
    pub fn to_term_decorator(&self) -> TermDecorator {
        match *self {
            Destination::Stdout => TermDecorator::new().stdout().build(),
            Destination::Stderr => TermDecorator::new().stderr().build(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TerminalLoggerConfig {
    // TODO: default
    pub level: Severity,
    pub format: Format,
    pub timezone: Timezone,
    pub destination: Destination,
}
impl Config for TerminalLoggerConfig {
    type Builder = TerminalLoggerBuilder;
    fn try_into_builder(self) -> Result<Self::Builder> {
        let mut builder = TerminalLoggerBuilder::new();
        builder.level(self.level);
        builder.format(self.format);
        builder.timezone(self.timezone);
        builder.destination(self.destination);
        Ok(builder)
    }
}
