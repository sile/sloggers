//! Null logger.
use crate::{Build, Config, Result};
use serde::{Deserialize, Serialize};
use slog::{Discard, Logger};

/// Null logger builder.
///
/// This will create a logger which discards all log records.
#[derive(Debug)]
pub struct NullLoggerBuilder;
impl Build for NullLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let logger = Logger::root(Discard, o!());
        Ok(logger)
    }
}

/// The configuration of `NullLoggerBuilder`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullLoggerConfig {}
impl Config for NullLoggerConfig {
    type Builder = NullLoggerBuilder;
    fn try_to_builder(&self) -> Result<Self::Builder> {
        Ok(NullLoggerBuilder)
    }
}
