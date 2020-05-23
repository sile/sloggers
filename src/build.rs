use crate::file::FileLoggerBuilder;
use crate::null::NullLoggerBuilder;
use crate::terminal::TerminalLoggerBuilder;
use crate::Result;
use slog::Logger;

/// This trait allows to build a logger instance.
pub trait Build {
    /// Builds a logger.
    fn build(&self) -> Result<Logger>;
}

/// Logger builder.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
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
