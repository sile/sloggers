use slog::{Drain, Logger};
use slog_term::Decorator;
use std::fmt::Debug;

use file::FileLoggerBuilder;
use null::NullLoggerBuilder;
use terminal::TerminalLoggerBuilder;
use Result;

/// This trait allows to build a logger instance.
pub trait Build {
    /// Builds a logger.
    fn build(&self) -> Result<Logger>;
}

pub trait BuildWithCustomFormat {
    type Decorator: Decorator;

    fn build_with_custom_format<F, D>(&self, f: F) -> Result<Logger>
    where
        F: FnOnce(Self::Decorator) -> Result<D>,
        D: Drain + Send + 'static,
        D::Err: Debug;
}

/// Logger builder.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum LoggerBuilder {
    /// File logger.
    File(FileLoggerBuilder),

    /// Null logger.
    Null(NullLoggerBuilder),

    /// Terminal logger.
    Terminal(TerminalLoggerBuilder),
}
impl Build for LoggerBuilder {
    fn build(&self) -> Result<Logger> {
        match *self {
            LoggerBuilder::File(ref b) => track!(b.build()),
            LoggerBuilder::Null(ref b) => track!(b.build()),
            LoggerBuilder::Terminal(ref b) => track!(b.build()),
        }
    }
}
