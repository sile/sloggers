use slog::{Logger, Discard};

use {Result, Build};

#[derive(Debug)]
pub struct NullLoggerBuilder;
impl Build for NullLoggerBuilder {
    fn build(&self) -> Result<Logger> {
        let logger = Logger::root(Discard, o!());
        Ok(logger)
    }
}
