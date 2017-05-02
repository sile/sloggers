use slog::{Logger, Discard};

use {Result, Build, Config};

#[derive(Debug)]
pub struct NullLoggerBuilder;
impl Build for NullLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let logger = Logger::root(Discard, o!());
        Ok(logger)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullLoggerConfig {}
impl Config for NullLoggerConfig {
    type Builder = NullLoggerBuilder;
    fn try_into_builder(self) -> Result<Self::Builder> {
        Ok(NullLoggerBuilder)
    }
}
