use serde::{Serialize, Deserialize};

use {Result, Build, LoggerBuilder};
use file::FileLoggerConfig;
use null::NullLoggerConfig;
use terminal::TerminalLoggerConfig;

/// Configuration of a logger builder.
pub trait Config: Sized + Serialize + for<'a> Deserialize<'a> {
    /// Logger builder.
    type Builder: Build;

    /// Makes a logger builder associated with this configuration.
    fn try_into_builder(self) -> Result<Self::Builder>;
}

/// The configuration of `LoggerBuilder`.
///
/// # Examples
///
/// Null logger.
///
/// ```
/// extern crate sloggers;
/// extern crate tomlconv;
///
/// use sloggers::{Config, LoggerConfig};
/// use tomlconv::FromToml;
///
/// # fn main() {
/// let toml = r#"
/// type = "null"
/// "#;
/// let _config = LoggerConfig::from_toml_str(toml).unwrap();
/// # }
/// ```
///
/// Terminal logger.
///
/// ```
/// extern crate sloggers;
/// extern crate tomlconv;
///
/// use sloggers::{Config, LoggerConfig};
/// use tomlconv::FromToml;
///
/// # fn main() {
/// let toml = r#"
/// type = "terminal"
/// level = "warning"
/// "#;
/// let _config = LoggerConfig::from_toml_str(toml).unwrap();
/// # }
/// ```
///
/// File logger.
///
/// ```
/// extern crate sloggers;
/// extern crate tomlconv;
///
/// use sloggers::{Config, LoggerConfig};
/// use tomlconv::FromToml;
///
/// # fn main() {
/// let toml = r#"
/// type = "file"
/// path = "/path/to/file.log"
/// timezone = "utc"
/// "#;
/// let _config = LoggerConfig::from_toml_str(toml).unwrap();
/// # }
/// ```
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LoggerConfig {
    #[serde(rename = "file")]
    File(FileLoggerConfig),

    #[serde(rename = "null")]
    Null(NullLoggerConfig),

    #[serde(rename = "terminal")]
    Terminal(TerminalLoggerConfig),
}
impl Config for LoggerConfig {
    type Builder = LoggerBuilder;
    fn try_into_builder(self) -> Result<Self::Builder> {
        match self {
            LoggerConfig::File(c) => track!(c.try_into_builder()).map(LoggerBuilder::File),
            LoggerConfig::Null(c) => track!(c.try_into_builder()).map(LoggerBuilder::Null),
            LoggerConfig::Terminal(c) => track!(c.try_into_builder()).map(LoggerBuilder::Terminal),
        }
    }
}
