use slog::Logger;

use Result;
use null::NullLoggerBuilder;
use terminal::TerminalLoggerBuilder;

pub trait Build {
    fn build(&self) -> Result<Logger>;
}

#[derive(Debug)]
pub struct LoggerBuilder;
impl LoggerBuilder {
    pub fn null() -> NullLoggerBuilder {
        NullLoggerBuilder
    }
    pub fn terminal() -> TerminalLoggerBuilder {
        TerminalLoggerBuilder::new()
    }
    // TODO: from_config
}
